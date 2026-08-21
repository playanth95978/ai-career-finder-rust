import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IOfferPositioning } from '../offer-positioning.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../offer-positioning.test-samples';

import { OfferPositioningService, RestOfferPositioning } from './offer-positioning.service';

const requireRestSample: RestOfferPositioning = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
};

describe('OfferPositioning Service', () => {
  let service: OfferPositioningService;
  let httpMock: HttpTestingController;
  let expectedResult: IOfferPositioning | IOfferPositioning[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(OfferPositioningService);
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

    it('should create a OfferPositioning', () => {
      const offerPositioning = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(offerPositioning).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a OfferPositioning', () => {
      const offerPositioning = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(offerPositioning).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a OfferPositioning', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of OfferPositioning', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a OfferPositioning', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addOfferPositioningToCollectionIfMissing', () => {
      it('should add a OfferPositioning to an empty array', () => {
        const offerPositioning: IOfferPositioning = sampleWithRequiredData;
        expectedResult = service.addOfferPositioningToCollectionIfMissing([], offerPositioning);
        expect(expectedResult).toEqual([offerPositioning]);
      });

      it('should not add a OfferPositioning to an array that contains it', () => {
        const offerPositioning: IOfferPositioning = sampleWithRequiredData;
        const offerPositioningCollection: IOfferPositioning[] = [
          {
            ...offerPositioning,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addOfferPositioningToCollectionIfMissing(offerPositioningCollection, offerPositioning);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a OfferPositioning to an array that doesn't contain it", () => {
        const offerPositioning: IOfferPositioning = sampleWithRequiredData;
        const offerPositioningCollection: IOfferPositioning[] = [sampleWithPartialData];
        expectedResult = service.addOfferPositioningToCollectionIfMissing(offerPositioningCollection, offerPositioning);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(offerPositioning);
      });

      it('should add only unique OfferPositioning to an array', () => {
        const offerPositioningArray: IOfferPositioning[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const offerPositioningCollection: IOfferPositioning[] = [sampleWithRequiredData];
        expectedResult = service.addOfferPositioningToCollectionIfMissing(offerPositioningCollection, ...offerPositioningArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const offerPositioning: IOfferPositioning = sampleWithRequiredData;
        const offerPositioning2: IOfferPositioning = sampleWithPartialData;
        expectedResult = service.addOfferPositioningToCollectionIfMissing([], offerPositioning, offerPositioning2);
        expect(expectedResult).toEqual([offerPositioning, offerPositioning2]);
      });

      it('should accept null and undefined values', () => {
        const offerPositioning: IOfferPositioning = sampleWithRequiredData;
        expectedResult = service.addOfferPositioningToCollectionIfMissing([], null, offerPositioning, undefined);
        expect(expectedResult).toEqual([offerPositioning]);
      });

      it('should return initial array if no OfferPositioning is added', () => {
        const offerPositioningCollection: IOfferPositioning[] = [sampleWithRequiredData];
        expectedResult = service.addOfferPositioningToCollectionIfMissing(offerPositioningCollection, undefined, null);
        expect(expectedResult).toEqual(offerPositioningCollection);
      });
    });

    describe('compareOfferPositioning', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareOfferPositioning(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 28017 };
        const entity2 = null;

        const compareResult1 = service.compareOfferPositioning(entity1, entity2);
        const compareResult2 = service.compareOfferPositioning(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 28017 };
        const entity2 = { id: 9189 };

        const compareResult1 = service.compareOfferPositioning(entity1, entity2);
        const compareResult2 = service.compareOfferPositioning(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 28017 };
        const entity2 = { id: 28017 };

        const compareResult1 = service.compareOfferPositioning(entity1, entity2);
        const compareResult2 = service.compareOfferPositioning(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
