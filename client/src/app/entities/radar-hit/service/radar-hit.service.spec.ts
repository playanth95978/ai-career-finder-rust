import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IRadarHit } from '../radar-hit.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../radar-hit.test-samples';

import { RadarHitService, RestRadarHit } from './radar-hit.service';

const requireRestSample: RestRadarHit = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
};

describe('RadarHit Service', () => {
  let service: RadarHitService;
  let httpMock: HttpTestingController;
  let expectedResult: IRadarHit | IRadarHit[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(RadarHitService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  describe('Service methods', () => {
    it('should find an element', () => {
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.find(123).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should create a RadarHit', () => {
      const radarHit = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(radarHit).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a RadarHit', () => {
      const radarHit = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(radarHit).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a RadarHit', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of RadarHit', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a RadarHit', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addRadarHitToCollectionIfMissing', () => {
      it('should add a RadarHit to an empty array', () => {
        const radarHit: IRadarHit = sampleWithRequiredData;
        expectedResult = service.addRadarHitToCollectionIfMissing([], radarHit);
        expect(expectedResult).toEqual([radarHit]);
      });

      it('should not add a RadarHit to an array that contains it', () => {
        const radarHit: IRadarHit = sampleWithRequiredData;
        const radarHitCollection: IRadarHit[] = [
          {
            ...radarHit,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addRadarHitToCollectionIfMissing(radarHitCollection, radarHit);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a RadarHit to an array that doesn't contain it", () => {
        const radarHit: IRadarHit = sampleWithRequiredData;
        const radarHitCollection: IRadarHit[] = [sampleWithPartialData];
        expectedResult = service.addRadarHitToCollectionIfMissing(radarHitCollection, radarHit);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(radarHit);
      });

      it('should add only unique RadarHit to an array', () => {
        const radarHitArray: IRadarHit[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const radarHitCollection: IRadarHit[] = [sampleWithRequiredData];
        expectedResult = service.addRadarHitToCollectionIfMissing(radarHitCollection, ...radarHitArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const radarHit: IRadarHit = sampleWithRequiredData;
        const radarHit2: IRadarHit = sampleWithPartialData;
        expectedResult = service.addRadarHitToCollectionIfMissing([], radarHit, radarHit2);
        expect(expectedResult).toEqual([radarHit, radarHit2]);
      });

      it('should accept null and undefined values', () => {
        const radarHit: IRadarHit = sampleWithRequiredData;
        expectedResult = service.addRadarHitToCollectionIfMissing([], null, radarHit, undefined);
        expect(expectedResult).toEqual([radarHit]);
      });

      it('should return initial array if no RadarHit is added', () => {
        const radarHitCollection: IRadarHit[] = [sampleWithRequiredData];
        expectedResult = service.addRadarHitToCollectionIfMissing(radarHitCollection, undefined, null);
        expect(expectedResult).toEqual(radarHitCollection);
      });
    });

    describe('compareRadarHit', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareRadarHit(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 25582 };
        const entity2 = null;

        const compareResult1 = service.compareRadarHit(entity1, entity2);
        const compareResult2 = service.compareRadarHit(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 25582 };
        const entity2 = { id: 20377 };

        const compareResult1 = service.compareRadarHit(entity1, entity2);
        const compareResult2 = service.compareRadarHit(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 25582 };
        const entity2 = { id: 25582 };

        const compareResult1 = service.compareRadarHit(entity1, entity2);
        const compareResult2 = service.compareRadarHit(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
