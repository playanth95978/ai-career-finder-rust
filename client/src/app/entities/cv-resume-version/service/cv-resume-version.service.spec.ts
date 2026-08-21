import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { ICvResumeVersion } from '../cv-resume-version.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../cv-resume-version.test-samples';

import { CvResumeVersionService, RestCvResumeVersion } from './cv-resume-version.service';

const requireRestSample: RestCvResumeVersion = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
};

describe('CvResumeVersion Service', () => {
  let service: CvResumeVersionService;
  let httpMock: HttpTestingController;
  let expectedResult: ICvResumeVersion | ICvResumeVersion[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(CvResumeVersionService);
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

    it('should create a CvResumeVersion', () => {
      const cvResumeVersion = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(cvResumeVersion).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a CvResumeVersion', () => {
      const cvResumeVersion = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(cvResumeVersion).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a CvResumeVersion', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of CvResumeVersion', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a CvResumeVersion', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addCvResumeVersionToCollectionIfMissing', () => {
      it('should add a CvResumeVersion to an empty array', () => {
        const cvResumeVersion: ICvResumeVersion = sampleWithRequiredData;
        expectedResult = service.addCvResumeVersionToCollectionIfMissing([], cvResumeVersion);
        expect(expectedResult).toEqual([cvResumeVersion]);
      });

      it('should not add a CvResumeVersion to an array that contains it', () => {
        const cvResumeVersion: ICvResumeVersion = sampleWithRequiredData;
        const cvResumeVersionCollection: ICvResumeVersion[] = [
          {
            ...cvResumeVersion,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addCvResumeVersionToCollectionIfMissing(cvResumeVersionCollection, cvResumeVersion);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a CvResumeVersion to an array that doesn't contain it", () => {
        const cvResumeVersion: ICvResumeVersion = sampleWithRequiredData;
        const cvResumeVersionCollection: ICvResumeVersion[] = [sampleWithPartialData];
        expectedResult = service.addCvResumeVersionToCollectionIfMissing(cvResumeVersionCollection, cvResumeVersion);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(cvResumeVersion);
      });

      it('should add only unique CvResumeVersion to an array', () => {
        const cvResumeVersionArray: ICvResumeVersion[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const cvResumeVersionCollection: ICvResumeVersion[] = [sampleWithRequiredData];
        expectedResult = service.addCvResumeVersionToCollectionIfMissing(cvResumeVersionCollection, ...cvResumeVersionArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const cvResumeVersion: ICvResumeVersion = sampleWithRequiredData;
        const cvResumeVersion2: ICvResumeVersion = sampleWithPartialData;
        expectedResult = service.addCvResumeVersionToCollectionIfMissing([], cvResumeVersion, cvResumeVersion2);
        expect(expectedResult).toEqual([cvResumeVersion, cvResumeVersion2]);
      });

      it('should accept null and undefined values', () => {
        const cvResumeVersion: ICvResumeVersion = sampleWithRequiredData;
        expectedResult = service.addCvResumeVersionToCollectionIfMissing([], null, cvResumeVersion, undefined);
        expect(expectedResult).toEqual([cvResumeVersion]);
      });

      it('should return initial array if no CvResumeVersion is added', () => {
        const cvResumeVersionCollection: ICvResumeVersion[] = [sampleWithRequiredData];
        expectedResult = service.addCvResumeVersionToCollectionIfMissing(cvResumeVersionCollection, undefined, null);
        expect(expectedResult).toEqual(cvResumeVersionCollection);
      });
    });

    describe('compareCvResumeVersion', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareCvResumeVersion(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 17476 };
        const entity2 = null;

        const compareResult1 = service.compareCvResumeVersion(entity1, entity2);
        const compareResult2 = service.compareCvResumeVersion(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 17476 };
        const entity2 = { id: 118 };

        const compareResult1 = service.compareCvResumeVersion(entity1, entity2);
        const compareResult2 = service.compareCvResumeVersion(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 17476 };
        const entity2 = { id: 17476 };

        const compareResult1 = service.compareCvResumeVersion(entity1, entity2);
        const compareResult2 = service.compareCvResumeVersion(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
