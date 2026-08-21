import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { ICvResume } from '../cv-resume.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../cv-resume.test-samples';

import { CvResumeService, RestCvResume } from './cv-resume.service';

const requireRestSample: RestCvResume = {
  ...sampleWithRequiredData,
  createdAt: sampleWithRequiredData.createdAt?.toJSON(),
  updatedAt: sampleWithRequiredData.updatedAt?.toJSON(),
};

describe('CvResume Service', () => {
  let service: CvResumeService;
  let httpMock: HttpTestingController;
  let expectedResult: ICvResume | ICvResume[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(CvResumeService);
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

    it('should create a CvResume', () => {
      const cvResume = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(cvResume).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a CvResume', () => {
      const cvResume = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(cvResume).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a CvResume', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of CvResume', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a CvResume', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addCvResumeToCollectionIfMissing', () => {
      it('should add a CvResume to an empty array', () => {
        const cvResume: ICvResume = sampleWithRequiredData;
        expectedResult = service.addCvResumeToCollectionIfMissing([], cvResume);
        expect(expectedResult).toEqual([cvResume]);
      });

      it('should not add a CvResume to an array that contains it', () => {
        const cvResume: ICvResume = sampleWithRequiredData;
        const cvResumeCollection: ICvResume[] = [
          {
            ...cvResume,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addCvResumeToCollectionIfMissing(cvResumeCollection, cvResume);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a CvResume to an array that doesn't contain it", () => {
        const cvResume: ICvResume = sampleWithRequiredData;
        const cvResumeCollection: ICvResume[] = [sampleWithPartialData];
        expectedResult = service.addCvResumeToCollectionIfMissing(cvResumeCollection, cvResume);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(cvResume);
      });

      it('should add only unique CvResume to an array', () => {
        const cvResumeArray: ICvResume[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const cvResumeCollection: ICvResume[] = [sampleWithRequiredData];
        expectedResult = service.addCvResumeToCollectionIfMissing(cvResumeCollection, ...cvResumeArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const cvResume: ICvResume = sampleWithRequiredData;
        const cvResume2: ICvResume = sampleWithPartialData;
        expectedResult = service.addCvResumeToCollectionIfMissing([], cvResume, cvResume2);
        expect(expectedResult).toEqual([cvResume, cvResume2]);
      });

      it('should accept null and undefined values', () => {
        const cvResume: ICvResume = sampleWithRequiredData;
        expectedResult = service.addCvResumeToCollectionIfMissing([], null, cvResume, undefined);
        expect(expectedResult).toEqual([cvResume]);
      });

      it('should return initial array if no CvResume is added', () => {
        const cvResumeCollection: ICvResume[] = [sampleWithRequiredData];
        expectedResult = service.addCvResumeToCollectionIfMissing(cvResumeCollection, undefined, null);
        expect(expectedResult).toEqual(cvResumeCollection);
      });
    });

    describe('compareCvResume', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareCvResume(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 8461 };
        const entity2 = null;

        const compareResult1 = service.compareCvResume(entity1, entity2);
        const compareResult2 = service.compareCvResume(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 8461 };
        const entity2 = { id: 15106 };

        const compareResult1 = service.compareCvResume(entity1, entity2);
        const compareResult2 = service.compareCvResume(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 8461 };
        const entity2 = { id: 8461 };

        const compareResult1 = service.compareCvResume(entity1, entity2);
        const compareResult2 = service.compareCvResume(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
